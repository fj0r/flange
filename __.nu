const WORKDIR = path self .
const CFG = path self __.toml
const CONFIG = path self config.toml

export def receiver [] {
    let c = open $CFG
    http get $"http://($c.server.host)/admin/users"
}

def cmpl-act [] {
    [Message Layout test]
}

def cmpl-data [] {
    cd ([$WORKDIR data message] | path join)
    ls | get name
}

export def send [
    file:string@cmpl-data
    --receiver(-r): list<string@receiver> = []
    --sender(-s): string = 'unknown'
    --patch(-p): record = {}
] {
    let c = open $CFG
    let d = open ([$WORKDIR data message $file] | path join)
    let host = $"http://($c.server.host)/admin/message"
    let data = $d | merge deep $patch
    print $"(ansi grey)($data | to yaml)(ansi reset)"
    http post --content-type application/json $host $data
}

export def 'dev serve' [] {
    $env.RUST_BACKTRACE = 1
    #$env.APP_KAFKA_ENABLE = 1
    cargo run
}

export def 'dev client' [] {
    let c = open $CFG
    websocat $"ws://($c.server.host)/channel"
}

export def 'dev test' [] {
    let ji = job spawn { dev serve }
    sleep 1sec
    do -i {
        dev client
    }
    job kill $ji
}


export def 'rpk send' [
    data
    --partition:int=0
    --topic(-t):string@"rpk topic list"
    --patch: record = {}
] {
    let c = open $CFG
    let data = { records: ($data | merge deep $patch | wrap value | insert partition $partition) } | to json -r
    http post -H [
        Content-Type application/vnd.kafka.json.v2+json
    ] $"http://($c.redpanda.admin)/topics/($topic)" $data
}

export def 'rpk subscribe' [topic:string@"rpk topic list"] {
    let c = open $CFG
    let data = { topics: [$topic] } | to json -r
    curl -sL $"http://($c.redpanda.admin)/topics/($topic)/partitions/0/records?offset=0" -H "Content-Type: application/vnd.kafka.json.v2+json" --data $data
}

export def 'rpk consume' [topic:string@"rpk topic list"] {
    mut args = [exec -it redpanda]
    $args ++= [rpk topic consume $topic]
    ^$env.CONTCTL ...$args
}

export def 'rpk group list' [] {
    mut args = [exec -it redpanda]
    $args ++= [rpk group list]
    ^$env.CONTCTL ...$args | from ssv
}

export def 'rpk group delete' [group:string@"rpk group list"] {
    mut args = [exec -it redpanda]
    $args ++= [rpk group delete $group]
    ^$env.CONTCTL ...$args
}

export def 'rpk topic list' [] {
    let c = open $CFG
    http get $"http://($c.redpanda.admin)/topics" | from json
}

export def 'rpk topic create' [name:string] {
    mut args = [exec -t redpanda]
    $args ++= [rpk topic create $name]
    ^$env.CONTCTL ...$args
}

export def 'rpk topic delete' [name:string@'rpk topic list'] {
    mut args = [exec -t redpanda]
    $args ++= [rpk topic delete $name]
    ^$env.CONTCTL ...$args
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

export def 'rpk test' [] {
    rpk up
    let s = open $CONFIG
    rpk topic create $s.queue.event.topic
    rpk topic create $s.queue.push.topic.0
    rpk send --topic event (open data/message/event.yaml)
    # for i in 1..100 {
    #     rpk send --topic event $i
    # }
    # rpk consume event
}
