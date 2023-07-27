use twilight_http::request::channel::webhook::ExecuteWebhook;
use twilight_model::channel::Channel;

/// Utility functions to execute webhooks
pub trait ExecuteWebhookExt {
    /// If the channel is a thread channel, execute the webhook in it
    #[must_use]
    fn in_channel(self, channel: &Channel) -> Self;
}

impl ExecuteWebhookExt for ExecuteWebhook<'_> {
    fn in_channel(self, channel: &Channel) -> Self {
        if channel.kind.is_thread() {
            self.thread_id(channel.id)
        } else {
            self
        }
    }
}
