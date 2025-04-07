def cmpl-sender [] {
    http get http://localhost:3000/admin/users
}

def cmpl-act [] {
    [Message Layout test]
}

export def send [receiver:string@cmpl-sender
    message
    --sender: string = 'unknown'
    --act: string@cmpl-act = 'Message'
] {
    let host = "http://localhost:3000/admin/message"
    let data = {
        receiver: [$receiver],
        message: {
            act: $act,
            user: $sender,
            content: $message
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
