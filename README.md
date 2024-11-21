## Necessary Knowledge

```
---compile code
cargo test
cargo build --release
```

## Recompile & Run vChain+

##### gen_key

```
./gen_key -q 1024 -o path/to/pk
```

##### build_chain

```
./build_chain -t 2 --id-fanout 4 -b 4 -m 1023 -d 1 -k path/to/pk -i ../../data/dataset/eth.dat -r path/to/build_time.json -o path/to/output_db
```

##### query

```
./query -e -n -k path/to/pk -i path/to/output_db/ -q path/to/query.json -r path/to/result/process_time.json -v 2
```

##### query.json

```
[
{
    "start_blk": 1,
    "end_blk": 20000,
    "range": [
      [
        250000,
        250001
      ]
    ],
    "keyword_exp": {
      "or": [
        {
          "input": "'0xa12431d0b9db640034b0cdfceef9cce161e62be4'"
        },
        {
          "input": "'0xa12431d0b9db640034b0cdfceef9cce161e62be4'"
        }
      ]
    }
}
]
```



## SimChain

#### Input format

```
block_id [address] {in/out, amount, timestamp}
```

For example

```
1 [muhtvdmsnbQEPFuEmxcChX58fGvXaaUoVt] {in, 50, 1571443461}
1 [mwhtvdmsnbQEPFuEmxcChX58fGvXaaUoVt] {in, 50, 1571443461}
1 [mvbnrCX3bg1cDRUu8pkecrvP6vQkSLDSou] {out, 10, 1571443461}
```

### Build Chain

Run `simchain-build` to build the chain. The default value of learned index error bounds is set to be 5.

```
./simchain-build -i data/input.txt -d data/db
```

Run `simchain-build -h` for more info.

### Deploy Chain

Run `simchain-server` after `simchain-build` is taken.

```
./simchain-server -d data/db 
```

Simchain's port is set to 8000 on default.

### Service API

Use RESTFul API to inspect the blockchain.

```
GET /get/param
GET /get/blk_header/{id}
GET /get/blk_data/{id}
GET /get/tx/{id}
```

For example, if a server is running on port 8000 locally, then the get_param request will be as followed in Linux

```
curl -X GET http:127.0.0.1:8000/get/param
```

#### Query

API endpoint is:

```
POST /query
```

Parameters are followed with this request in JSON format

```
{
    "blk_addr":"1H5BckuQwEDdZXBjJsSswm4jm6sYgSrqUs",
    "time_stamp": [
        1655512688,
        1655512688
    ],
    "inter_index": true,
    "intra_index": true
}
```

The response is a JSON object like

```
{
    "result": ...,
    "res_sigs": ...,
    "query_param": ...,
    "query_time_ms": ...,
    "use_inter_index": ...,
    "use_intra_index": ...
}
```

#### Verify

API endpoint is:

```
POST /verify
```

Parameters are followed with this request in JSON format, which is the response of Verify process

```
{
    "result": ...,
    "res_sigs": ...,
    "query_param": ...,
    "query_time_ms": ...,
    "use_inter_index": ...,
    "use_intra_index": ...
}
```

The response is a JSON object like:

```
{
    "pass": ...,
    "fail_detail": ...,
    "verify_time_in_ms": ...
}
```

