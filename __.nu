const CONFIG = path self __.toml

def cmpl-sender [] {
    let c = open $CONFIG
    http get $"http://($c.server.host)/admin/users"
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
    let c = open $CONFIG
    let host = $"http://($c.server.host)/admin/message"
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
    let c = open $CONFIG
    $env.APP_KAFKA_ENABLE = 1
    $env.APP_KAFKA_CONSUMER_TOPIC = 'chat'
    $env.APP_KAFKA_PRODUCER_TOPIC = 'ai'
    let ji = job spawn { cargo run }
    sleep 1sec
    do -i {
        websocat $"ws://($c.server.host)/channel"
    }
    job kill $ji
}


export def 'rpk send' [...data -p:int=0 --topic(-t):string = 'logs'] {
    let c = open $CONFIG
    let data = { records: ($data | wrap value | insert partition $p) } | to json -r
    curl -sL -X POST $"http://($c.redpanda.admin)/topics/($topic)" -H "Content-Type: application/vnd.kafka.json.v2+json" --data $data
}

export def 'rpk send-log' [num:int=0, --batch:int=10, --topic(-t):string = 'logs'] {
    cat data/ingress-log.jsonl | from json -o | range $num..($num + $batch)
    | rpk send --topic $topic ...$in
}

export def 'rpk subscribe' [topic:string="logs"] {
    let c = open $CONFIG
    let data = { topics: [$topic] } | to json -r
    curl -sL $"http://($c.redpanda.admin)/topics/($topic)/partitions/0/records?offset=0&timeout=1000&max_bytes=100000" -H "Content-Type: application/vnd.kafka.json.v2+json" --data $data
}

export def 'rpk up' [
    --dry-run
] {
    let image = 'redpandadata/redpanda:latest'
    mut args = [run -d --name redpanda]
    let ports = {
        '18081': 18081
        '18082': 18082
        '19092': 19092
        '19644': 9644
    }
    for i in ($ports | transpose k v) {
        $args ++= [-p $"($i.k):($i.v)"]
    }
    let envs = {
    }
    for i in ($envs | transpose k v) {
        $args ++= [-e $"($i.k)=($i.v)"]
    }
    $args ++= [$image]
    $args ++= [
        redpanda
        start
        --kafka-addr
        'internal://0.0.0.0:9092,external://0.0.0.0:19092'
        --advertise-kafka-addr
        'internal://127.0.0.1:9092,external://localhost:19092'
        --pandaproxy-addr
        'internal://0.0.0.0:8082,external://0.0.0.0:18082'
        --advertise-pandaproxy-addr
        'internal://127.0.0.1:8082,external://localhost:18082'
        --schema-registry-addr
        'internal://0.0.0.0:8081,external://0.0.0.0:18081'
        --rpc-addr
        localhost:33145
        --advertise-rpc-addr
        localhost:33145
        --mode
        dev-container
        --smp 1
        --default-log-level=info
    ]
    if $dry_run {
        print $"($env.CONTCTL) ($args | str join ' ')"
    } else {
        ^$env.CONTCTL ...$args
    }
}
