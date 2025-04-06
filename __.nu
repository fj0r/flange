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
