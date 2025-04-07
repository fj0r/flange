def cmpl-sender [] {
    http get http://localhost:3000/admin/users
}

export def send [receiver:string@cmpl-sender
    message
    --sender: string = 'unknown'
] {
    let host = "http://localhost:3000/admin/message"
    let data = {
        receiver: [$receiver],
        content: {
            user: $sender, message: $message
        }
    }
    http post --content-type application/json $host $data
}

export def test [] {
    let ji = job spawn { cargo run }
    sleep 1sec
    websocat ws://localhost:3000/channel
    job kill $ji
}
