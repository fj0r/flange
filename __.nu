def cmpl-sender [] {
    http get http://localhost:3000/admin/users
}

def cmpl-act [] {
    [Message Layout test]
}

export def send [
    message
    --receiver(-r): list<string@cmpl-sender> = []
    --sender(-s): string = 'unknown'
    --act(-a): string@cmpl-act = 'Message'
] {
    let host = "http://localhost:3000/admin/message"
    let data = {
        receiver: $receiver,
        message: {
            act: $act,
            user: $sender,
            content: $message
        }
    }
    http post --content-type application/json $host $data
}

export def test [] {
    $env.APP_KAFKA_ENABLE = 1
    $env.APP_KAFKA_CONSUMER_TOPIC = 'chat'
    $env.APP_KAFKA_PRODUCER_TOPIC = 'ai'
    let ji = job spawn { cargo run }
    sleep 1sec
    websocat ws://localhost:3000/channel
    job kill $ji
}
