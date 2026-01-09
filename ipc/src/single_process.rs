//! Tools to use a single process sending data to it and closing other processes.

use ::core::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::ControlFlow,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    time::Duration,
};
use ::std::{
    sync::{Arc, Weak},
    time::Instant,
};

use ::iceoryx2::{
    node::{NodeCreationFailure, NodeWaitFailure},
    port::{
        LoanError, ReceiveError, SendError,
        client::RequestSendError,
        listener::ListenerCreateError,
        notifier::{NotifierCreateError, NotifierNotifyError},
        publisher::PublisherCreateError,
        subscriber::{Subscriber, SubscriberCreateError},
    },
    prelude::*,
    service::{
        builder::publish_subscribe::PublishSubscribeOpenOrCreateError,
        service_name::ServiceNameError,
    },
};
use iceoryx2::service::builder::event::EventOpenOrCreateError;

/// Handle to subscriber thread.
#[derive(Debug, Clone)]
pub struct SubscriberHandle {
    /// Id used for hashing and comparison.
    subscriber_id: u64,
    /// Keep alive variable, setting it to false will
    /// kill subscriber.
    keep_alive: Weak<AtomicBool>,
}

impl SubscriberHandle {
    /// Get a new instance with keep_alive arc.
    fn new() -> (Self, Arc<AtomicBool>) {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let subscriber_id = COUNTER.fetch_add(1, Relaxed);
        let keep_alive_strong = Arc::new(AtomicBool::new(true));
        let keep_alive = Arc::downgrade(&keep_alive_strong);

        (
            Self {
                subscriber_id,
                keep_alive,
            },
            keep_alive_strong,
        )
    }

    /// Check if the subscriber is or is set to be closed.
    pub fn is_closed(&self) -> bool {
        let Some(keep_alive) = self.keep_alive.upgrade() else {
            return true;
        };

        !keep_alive.load(Relaxed)
    }

    /// Set the subscriber to be closed.
    pub fn close(&self) {
        if let Some(keep_alive) = self.keep_alive.upgrade() {
            keep_alive.store(false, Relaxed);
        }
    }
}

impl Hash for SubscriberHandle {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        self.subscriber_id.hash(state);
    }
}

impl PartialEq for SubscriberHandle {
    fn eq(&self, other: &Self) -> bool {
        self.subscriber_id == other.subscriber_id
    }
}

impl Eq for SubscriberHandle {}

/// Event used for notifying subscriber.
const NOTIFY_EVENT: EventId = EventId::new(11);

/// Event used for notifying subscriber.
const REPLACE_EVENT: EventId = EventId::new(13);

/// Type alias for port factory.
type PublishSubscribePortFactory<M> =
    ::iceoryx2::service::port_factory::publish_subscribe::PortFactory<
        ipc_threadsafe::Service,
        M,
        (),
    >;

/// Type alias for port factory.
type EventPortFactory =
    iceoryx2::service::port_factory::event::PortFactory<ipc_threadsafe::Service>;

/// Alias for event service.
type EventService = ::iceoryx2::service::port_factory::event::PortFactory<ipc_threadsafe::Service>;

/// Create ipc node.
fn build_node(name: &NodeName) -> Result<Node<ipc_threadsafe::Service>, NodeCreationFailure> {
    NodeBuilder::new()
        .name(name)
        .signal_handling_mode(SignalHandlingMode::Disabled)
        .create::<ipc_threadsafe::Service>()
}

/// Create publish subscribe service.
fn build_serice_<M>(
    name: &ServiceName,
    node: &Node<ipc_threadsafe::Service>,
) -> Result<PublishSubscribePortFactory<M>, PublishSubscribeOpenOrCreateError>
where
    M: Debug + ZeroCopySend,
{
    node.service_builder(name)
        .publish_subscribe::<M>()
        .max_subscribers(1)
        .open_or_create()
}

/// Create publish subscribe service.
fn build_service<M>(
    name: &ServiceName,
    node: &Node<ipc_threadsafe::Service>,
) -> Result<PublishSubscribePortFactory<M>, PublishSubscribeOpenOrCreateError>
where
    M: Debug + ZeroCopySend,
{
    build_serice_::<M>(name, node).or_else(|_| {
        if let Err(err) = Node::<ipc_threadsafe::Service>::list(
            ::iceoryx2::config::Config::global_config(),
            |node_state| {
                if let NodeState::<ipc_threadsafe::Service>::Dead(view) = node_state {
                    ::log::info!("cleanup of dead node {view:?}");
                    if let Err(err) = view.remove_stale_resources() {
                        ::log::warn!("could nod clean up stale resources, {err:?}");
                    }
                }
                CallbackProgression::Continue
            },
        ) {
            ::log::error!("failed to perform stale resource cleanup, {err}");
        }

        build_serice_(name, node)
    })
}

/// Create event service.
fn build_event_service(
    name: &ServiceName,
    node: &Node<ipc_threadsafe::Service>,
) -> Result<EventPortFactory, EventOpenOrCreateError> {
    node.service_builder(name).event().open_or_create()
}

/// Create subscriber thread.
fn create_subscriber_thread<M, E, S>(
    subscriber: Subscriber<ipc_threadsafe::Service, M, ()>,
    event_service: EventService,
    thread_name: String,
    mut receive: S,
) -> Result<SubscriberHandle, E>
where
    M: Debug + ZeroCopySend,
    E: 'static
        + Send
        + Sync
        + Display
        + From<::std::io::Error>
        + From<ListenerCreateError>
        + From<ReceiveError>,
    S: 'static + Send + FnMut(&M) -> Result<(), E>,
{
    let (handle, keep_alive) = SubscriberHandle::new();
    ::std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let receive_messages = || -> Result<(), E> {
                let listener = event_service.listener_builder().create()?;
                while keep_alive.load(Relaxed)
                    && listener
                        .timed_wait_all(
                            |event| {
                                if event == REPLACE_EVENT {
                                    ::log::info!("received replace event, exiting subscribe loop");
                                    keep_alive.store(false, Relaxed);
                                }
                            },
                            Duration::from_millis(200),
                        )
                        .is_ok()
                {
                    while let Some(message) = subscriber.receive()? {
                        ::log::info!("received ipc message");
                        receive(&message)?;
                    }
                }
                drop(subscriber);
                Ok(())
            };

            if let Err(err) = receive_messages() {
                ::log::error!("error receiving ipc messages\n{err}");
            }

            ::log::info!("closing ipc thread");
            keep_alive.store(false, Relaxed);
        })
        .map(|_| handle)
        .map_err(E::from)
}

/// Publish input to eventual subscribers.
fn publish_input<M, I, E>(
    node: Node<ipc_threadsafe::Service>,
    service: PublishSubscribePortFactory<M>,
    event_service: EventService,
    input: I,
) -> Result<(), E>
where
    M: 'static + Debug + ZeroCopySend,
    E: From<SendError>
        + From<RequestSendError>
        + From<PublisherCreateError>
        + From<NotifierCreateError>
        + From<LoanError>,
    I: FnOnce() -> Result<M, E>,
{
    let publisher = service.publisher_builder().create()?;
    let notifier = event_service
        .notifier_builder()
        .default_event_id(NOTIFY_EVENT)
        .create()?;

    let message = publisher.loan_uninit()?;
    let message = message.write_payload(input()?);
    message.send()?;
    ::log::info!("sent ipc message");
    let wait_result = if let Err(err) = notifier.notify() {
        ::log::error!("could not send notification event, {err}");
        node.wait(Duration::from_millis(200))
    } else {
        node.wait(Duration::from_millis(50))
    };
    if let Err(err) = wait_result {
        ::log::warn!("after-publish wait interrupted, {err}");
    }
    Ok(())
}

/// Error returned when subscribe_only times out.
#[derive(Debug, ::thiserror::Error)]
#[error("subscribe_only reached timeout of {timeout:.4}s", timeout = timeout.as_secs_f64())]
pub struct SubscribeOnlyTimeoutError {
    /// Timeout that was reached.
    pub timeout: Duration,
}

/// Setup ipc for subscribing only requesting any prior subscriber to stop subscribing.
///
/// # Errors
/// If ipc cannot be setup, either due to invalid preconditions
/// or the timout running out whilst asking other subscribers to step down.
fn subscribe_only_<M, R, T, E>(
    node_name: &'static str,
    service_name: &'static str,
    thread_name: T,
    receive: R,
    timeout: Duration,
) -> Result<SubscriberHandle, E>
where
    M: 'static + Debug + ZeroCopySend,
    R: 'static + Send + FnMut(&M) -> Result<(), E>,
    T: FnOnce() -> String,
    E: 'static
        + Display
        + Send
        + Sync
        + From<::std::io::Error>
        + From<EventOpenOrCreateError>
        + From<ListenerCreateError>
        + From<NodeCreationFailure>
        + From<PublishSubscribeOpenOrCreateError>
        + From<ReceiveError>
        + From<SemanticStringError>
        + From<ServiceNameError>
        + From<SubscriberCreateError>
        + From<NotifierCreateError>
        + From<NotifierNotifyError>
        + From<NodeWaitFailure>
        + From<SubscribeOnlyTimeoutError>,
{
    let node_name = NodeName::new(node_name)?;
    let service_name = ServiceName::new(service_name)?;

    let node = build_node(&node_name)?;
    let service = build_service::<M>(&service_name, &node)?;
    let event_service = build_event_service(&service_name, &node)?;

    match service.subscriber_builder().create() {
        Ok(subscriber) => {
            create_subscriber_thread(subscriber, event_service, thread_name(), receive)
        }
        Err(SubscriberCreateError::ExceedsMaxSupportedSubscribers) => {
            let timeout_instant = Instant::now() + timeout;
            let mut max_sleep = 0.002f64;
            let notifier = event_service
                .notifier_builder()
                .default_event_id(REPLACE_EVENT)
                .create()?;
            loop {
                max_sleep = (0.1f64).min(max_sleep * 2.0);
                notifier.notify()?;
                node.wait(Duration::from_secs_f64(::rand::random_range(
                    0.0..=max_sleep,
                )))?;

                return match service.subscriber_builder().create() {
                    Ok(subscriber) => {
                        create_subscriber_thread(subscriber, event_service, thread_name(), receive)
                    }
                    Err(SubscriberCreateError::ExceedsMaxSupportedSubscribers) => {
                        if Instant::now() > timeout_instant {
                            return Err(SubscribeOnlyTimeoutError { timeout }.into());
                        }
                        continue;
                    }
                    Err(err) => Err(err.into()),
                };
            }
        }
        Err(err) => Err(err.into()),
    }
}

/// Setup ipc for single process.
fn single_process_<M, I, R, T, E>(
    node_name: &'static str,
    service_name: &'static str,
    thread_name: T,
    input: I,
    receive: R,
) -> Result<ControlFlow<(), SubscriberHandle>, E>
where
    M: 'static + Debug + ZeroCopySend,
    R: 'static + Send + FnMut(&M) -> Result<(), E>,
    I: FnOnce() -> Result<M, E>,
    T: FnOnce() -> String,
    E: 'static
        + Send
        + Sync
        + Display
        + From<::std::io::Error>
        + From<EventOpenOrCreateError>
        + From<ListenerCreateError>
        + From<LoanError>
        + From<NodeCreationFailure>
        + From<NotifierCreateError>
        + From<PublishSubscribeOpenOrCreateError>
        + From<PublisherCreateError>
        + From<ReceiveError>
        + From<RequestSendError>
        + From<SemanticStringError>
        + From<SendError>
        + From<ServiceNameError>
        + From<SubscriberCreateError>,
{
    let node_name = NodeName::new(node_name)?;
    let service_name = ServiceName::new(service_name)?;

    let node = build_node(&node_name)?;
    let service = build_service::<M>(&service_name, &node)?;
    let event_service = build_event_service(&service_name, &node)?;

    match service.subscriber_builder().create() {
        Ok(subscriber) => {
            create_subscriber_thread(subscriber, event_service, thread_name(), receive)
                .map(ControlFlow::Continue)
        }
        Err(SubscriberCreateError::ExceedsMaxSupportedSubscribers) => {
            publish_input::<M, I, E>(node, service, event_service, input)?;
            Ok(ControlFlow::Break(()))
        }
        Err(err) => Err(err.into()),
    }
}

/// Setup ipc for subscribing only requesting any prior subscriber to stop subscribing.
///
/// # Errors
/// If ipc cannot be setup, either due to invalid preconditions
/// or the timout running out whilst asking other subscribers to step down.
#[bon::builder]
#[builder(finish_fn = setup)]
pub fn subscribe_only<M, R, T, E>(
    /// Name to give ipc node.
    node_name: &'static str,
    /// Name to give single_process service.
    #[builder(default = "single_process")]
    service_name: &'static str,
    /// Name of subscriber thread.
    thread_name: Option<T>,
    /// Recevier for inputs sent from other processes if subscriber.
    receive: R,
    /// For how long to attempt to replace other subscribers.
    #[builder(default = Duration::from_millis(200))]
    timeout: Duration,
) -> Result<SubscriberHandle, E>
where
    M: 'static + Debug + ZeroCopySend,
    R: 'static + Send + FnMut(&M) -> Result<(), E>,
    T: FnOnce() -> String,
    E: 'static
        + Display
        + Send
        + Sync
        + From<::std::io::Error>
        + From<EventOpenOrCreateError>
        + From<ListenerCreateError>
        + From<NodeCreationFailure>
        + From<PublishSubscribeOpenOrCreateError>
        + From<ReceiveError>
        + From<SemanticStringError>
        + From<ServiceNameError>
        + From<SubscriberCreateError>
        + From<NotifierCreateError>
        + From<NotifierNotifyError>
        + From<NodeWaitFailure>
        + From<SubscribeOnlyTimeoutError>,
{
    subscribe_only_(
        node_name,
        service_name,
        move || {
            if let Some(thread_name) = thread_name {
                thread_name()
            } else {
                "single_process_subscriber".to_owned()
            }
        },
        receive,
        timeout,
    )
}

/// Setup ipc for single process.
///
/// # Errors
/// If ipc cannot be setup, in such a case no data
/// will have been sent to any eventual subscribers.
#[bon::builder]
#[builder(finish_fn = setup)]
pub fn single_process<M, I, R, T, E>(
    /// Name to give ipc node.
    node_name: &'static str,
    /// Name to give single_process service.
    #[builder(default = "single_process")]
    service_name: &'static str,
    /// Name of eventual subscriber thread.
    thread_name: Option<T>,
    /// Input to send if publisher.
    input: I,
    /// Recevier for inputs sent from other processes if subscriber.
    receive: R,
) -> Result<ControlFlow<(), SubscriberHandle>, E>
where
    M: 'static + Debug + ZeroCopySend,
    R: 'static + Send + FnMut(&M) -> Result<(), E>,
    I: FnOnce() -> Result<M, E>,
    T: FnOnce() -> String,
    E: 'static
        + Send
        + Sync
        + Display
        + From<::std::io::Error>
        + From<EventOpenOrCreateError>
        + From<ListenerCreateError>
        + From<LoanError>
        + From<NodeCreationFailure>
        + From<NotifierCreateError>
        + From<PublishSubscribeOpenOrCreateError>
        + From<PublisherCreateError>
        + From<ReceiveError>
        + From<RequestSendError>
        + From<SemanticStringError>
        + From<SendError>
        + From<ServiceNameError>
        + From<SubscriberCreateError>,
{
    single_process_(
        node_name,
        service_name,
        move || {
            if let Some(thread_name) = thread_name {
                thread_name()
            } else {
                "single_process_subscriber".to_owned()
            }
        },
        input,
        receive,
    )
}
