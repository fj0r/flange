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


export def 'rpk send' [...data -p:int=0 --topic(-t):string = 'logs'] {
    let data = { records: ($data | wrap value | insert partition $p) } | to json -r
    curl -sL -X POST $"http://172.178.5.123:28082/topics/($topic)" -H "Content-Type: application/vnd.kafka.json.v2+json" --data $data
}

export def 'rpk send-log' [num:int=0, --batch:int=10, --topic(-t):string = 'logs'] {
    cat data/ingress-log.jsonl | from json -o | range $num..($num + $batch)
    | rpk send --topic $topic ...$in
}

export def 'rpk subscribe' [topic:string="logs"] {
    let data = { topics: [$topic] } | to json -r
    curl -sL $"http://172.178.5.123:28082/topics/($topic)/partitions/0/records?offset=0&timeout=1000&max_bytes=100000" -H "Content-Type: application/vnd.kafka.json.v2+json" --data $data
}

export def 'rpk up' [
    --dry-run
] {
    let image = 'docker.redpanda.com/redpandadata/redpanda:v24.2.10'
    let ports = {
        '18081': 18081
        '18082': 18082
        '9092': 19092
    }
    mut args = [run --rm --name redpanda]
    for i in ($ports | transpose k v) {
        $args ++= [-p $"($i.k):($i.v)"]
    }
    $args ++= [$image]
    $args ++= [
      redpanda
      start
      --kafka-addr
      internal://0.0.0.0:9092,external://0.0.0.0:19092
      --advertise-kafka-addr
      internal://127.0.0.1:9092,external://172.178.5.123:19092
      --pandaproxy-addr
      internal://0.0.0.0:8082,external://0.0.0.0:18082
      --advertise-pandaproxy-addr
      internal://redpanda:8082,external://172.178.5.123:18082
      --schema-registry-addr
      internal://0.0.0.0:8081,external://0.0.0.0:18081
      #--rpc-addr redpanda:33145
      #--advertise-rpc-addr redpanda:33145
      --overprovisioned
      --smp 1
      --memory 1G
      --reserve-memory 0M
      --node-id 0
      --check=false
    ]
    if $dry_run {
        print $"($env.CONTCTL) ($args | str join ' ')"
    } else {
        let args = $args
        job spawn {
            ^$env.CONTCTL ...$args
        }
    }
}
