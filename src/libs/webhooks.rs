use std::fmt::Debug;

use super::settings::Webhook;
pub async fn handle_webhook<'a, T>(wh: &'a Webhook, msg: T) -> T
where
    T: Debug,
{
    println!("{:?}", wh);
    println!("{:?}", msg);
    return msg;
}
