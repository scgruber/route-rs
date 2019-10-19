use crate::link::{CloneLink, JoinLink, Link, LinkBuilder, PacketStream};
use crate::element::Element;
use crate::link::SyncLinkBuilder;

#[derive(default)]
pub struct MtransformNComposite<E: Element> {
    in_streams: Option<Vec<PacketStream<E::Input>>>,
    element: Option<E>,
    join_queue_capacity: usize,
    clone_queue_capacity: usize,
    num_egressors: Option<usize>,
}

impl<E: Element> MtransformNComposite<E> {
    pub fn new() -> Self {
        MtransformNComposite {
            in_streams: None,
            element: None,
            join_queue_capacity: 10,
            clone_queue_capacity: 10,
            num_egressors: None,
        }
    }

    /// Changes join_queue_capcity, default value is 10.
    /// Valid range is 1..=1000
    pub fn join_queue_capacity(self, queue_capacity: usize) -> Self {
        assert!(
            (1..=1000).contains(&queue_capacity),
            format!("join_queue_capacity: {} must be in range 1..=1000", queue_capacity)
        );

        MtransformNComposite {
            in_streams: self.in_streams,
            element: self.element,
            join_queue_capacity: queue_capacity,
            clone_queue_capacity: self.clone_queue_capacity,
            num_egressors: self.num_egressors,
        }
    }

    /// Changes tee_queue_capcity, default value is 10.
    /// Valid range is 1..=1000
    pub fn clone_queue_capacity(self, queue_capacity: usize) -> Self {
        assert!(
            (1..=1000).contains(&queue_capacity),
            format!("clone_queue_capacity: {} must be in range 1..=1000", queue_capacity)
        );

        MtransformNComposite {
            in_streams: self.in_streams,
            element: self.element,
            join_queue_capacity: self.join_queue_capacity,
            clone_queue_capacity: queue_capacity,
            num_egressors: self.num_egressors,
        }
    }

    pub fn num_egressors(self, num_egressors: usize) -> Self {
        assert!(
            num_egressors <= 1000,
            format!("compsite num_egressors: {} > 1000", num_egressors)
        );
        assert_ne!(num_egressors, 0, "num_egressors must be non-zero");

        MtoNComposite {
            in_streams: self.in_streams,
            element: self.element,
            join_queue_capacity: self.join_queue_capacity,
            clone_queue_capacity: self.clone_queue_capacity,
            num_egressors: Some(num_egressors),
        }
    }
}

impl<E:Element + Send + 'static> LinkBuilder<E::Input, E::Output> for MtransformNComposite<E> {
    fn ingressors(self, in_streams: Vec<PacketStream<E>>) -> Self {
        assert!(
            in_streams.len() > 1 && in_streams.len() <= 1000,
            format!("Input streams {} not in 2..=1000", in_streams.len())
        );
        MtransformNComposite {
            in_streams: Some(in_streams),
            element: self.element,
            join_queue_capacity: self.join_queue_capacity,
            clone_queue_capacity: self.clone_queue_capacity,
            num_egressors: self.num_egressors,
        }
    }

    fn build_link(self) -> Link<E::Output> {
        if self.in_streams.is_none() {
            panic!("Cannot build link! Missing input stream");
        } else if self.num_egressors.is_none() {
            panic!("Cannot build link! Missing number of num_egressors");
        } else if self.element.is_none() {
            panic!("Cannot build link! Missing element");
        } else {
            let (join_runnables, join_egressors) = JoinLink::new()
                .ingressors(self.in_streams.unwrap())
                .queue_capacity(self.join_queue_capacity)
                .build_link();

            let (sync_runnables, sync_egressors) = SyncLinkBuilder::new()
                .ingressors(join_egressors)
                .element(self.element)
                .build_link();
            
            let (mut clone_link_runnables, clone_link_egressors) = CloneLink::new()
                .ingressors(sync_egressors)
                .queue_capacity(self.clone_queue_capacity)
                .num_egressors(self.num_egressors.unwrap())
                .build_link();
            clone_link_runnables.extend(join_runnables);
            clone_link_runnables.extend(sync_runnables);
            
            (clone_link_runnables, clone_link_egressors)
        }
    }
}

impl<E: Element + Send + 'static> ElementLinkBuilder<E> for MtransformNComposite<E> {
    fn element(self, element: E) -> Self {
        MtransformNComposite {
            in_streams: self.in_streams,
            element: Some(element),
            join_queue_capacity: self.join_queue_capacity,
            clone_queue_capacity: self.clone_queue_capacity,
            num_egressors: self.num_egressors,
        }        
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::link::{LinkBuilder, TokioRunnable};
    use crate::utils::test::packet_collectors::ExhaustiveCollector;
    use crate::utils::test::packet_generators::immediate_stream;
    use crossbeam::crossbeam_channel;

    use futures::future::lazy;

    fn run_tokio(runnables: Vec<TokioRunnable>) {
        tokio::run(lazy(|| {
            for runnable in runnables {
                tokio::spawn(runnable);
            }
            Ok(())
        }));
    }

    #[test]
    fn MtransformN_Composite() {
        let packets = vec![0, 1, 2, 420, 1337, 3, 4, 5, 6, 7, 8, 9, 11];
        let number_num_egressors = 2;
        let packet_generator0 = immediate_stream(packets.clone());
        let packet_generator1 = immediate_stream(packets.clone());

        let mut input_streams: Vec<PacketStream<usize>> = Vec::new();
        input_streams.push(Box::new(packet_generator0));
        input_streams.push(Box::new(packet_generator1));

        let (mut runnables, mut egressors) = MtoNComposite::new()
            .num_egressors(number_num_egressors)
            .ingressors(input_streams)
            .build_link();

        let (s1, collector1_output) = crossbeam_channel::unbounded();
        let collector1 = ExhaustiveCollector::new(0, Box::new(egressors.pop().unwrap()), s1);
        runnables.push(Box::new(collector1));

        let (s0, collector0_output) = crossbeam_channel::unbounded();
        let collector0 = ExhaustiveCollector::new(0, Box::new(egressors.pop().unwrap()), s0);
        runnables.push(Box::new(collector0));

        run_tokio(runnables);

        let output0: Vec<_> = collector0_output.iter().collect();
        assert_eq!(output0.len(), packets.len() * 2);

        let output1: Vec<_> = collector1_output.iter().collect();
        assert_eq!(output1.len(), packets.len() * 2);
    }
}