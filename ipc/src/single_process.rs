//! Tools to use a single process sending data to it and closing other processes.

use ::core::{
    fmt::{Debug, Display},
    ops::ControlFlow,
    time::Duration,
};

use ::iceoryx2::{
    node::NodeCreationFailure,
    port::{
        LoanError, ReceiveError, SendError,
        client::RequestSendError,
        listener::ListenerCreateError,
        notifier::NotifierCreateError,
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

/// Event used for notifying subscriber.
const NOTIFY_EVENT: EventId = EventId::new(11);

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
) -> Result<(), E>
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
    ::std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let mut receive_messages = move || -> Result<(), E> {
                let listener = event_service.listener_builder().create()?;
                while listener
                    .timed_wait_all(|_| {}, Duration::from_millis(200))
                    .is_ok()
                {
                    while let Some(message) = subscriber.receive()? {
                        ::log::info!("received ipc message");
                        receive(&message)?;
                    }
                }
                Ok(())
            };

            if let Err(err) = receive_messages() {
                ::log::error!("error receiving ipc messages\n{err}");
            }

            ::log::info!("closing ipc thread");
        })?;
    Ok(())
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

/// Setup ipc for single process.
fn single_process_<M, I, R, T, E>(
    node_name: &'static str,
    service_name: &'static str,
    thread_name: T,
    input: I,
    receive: R,
) -> Result<ControlFlow<()>, E>
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
            create_subscriber_thread(subscriber, event_service, thread_name(), receive)?;
            Ok(ControlFlow::Continue(()))
        }
        Err(SubscriberCreateError::ExceedsMaxSupportedSubscribers) => {
            publish_input::<M, I, E>(node, service, event_service, input)?;
            Ok(ControlFlow::Break(()))
        }
        Err(err) => Err(err.into()),
    }
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
    service_name: &'static str,
    /// Name of eventual subscriber thread.
    thread_name: Option<T>,
    /// Input to send if publisher.
    input: I,
    /// Recevier for inputs sent from other processes if subscriber.
    receive: R,
) -> Result<ControlFlow<()>, E>
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
