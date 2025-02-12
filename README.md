## Run
```
cargo run --release
```

## Use
```
curl http://127.0.0.1:3000/lookup/<IP>
```

## Bench
```
ab -n 100000 -c 1000 http://127.0.0.1:3000/lookup_bench/<IP>
```

## Some stats
```
curl http://127.0.0.1:3000/stats
```

## Todo
[ ] add some in-memory MFU/MRU caching for responses
