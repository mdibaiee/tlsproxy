tlsproxy
=================

A most basic TLS man-in-the-middle forward proxy using [rustls](https://github.com/ctz/rustls) and [Tokio](tokio.rs).

This proxy allows you, given you have the certificate chain of the server, to replace byte sequences of length N with another byte sequence of length N in outbound requests.
For example, you could replace `https://testserver.com/?name=foo` with `?name=bar`
This is a toy, proof of concept project. It is not thoroughly tested and will have issues with a very high probability.

Sample usage:
```
$ cargo build && cargo run -- \
  --chaincert test-ca/end.fullchain \
  --key test-ca/end.key \
  --cacert test-ca/ca.cert \
  --replace 's/foo/bar' \
  --verbose
  
Listening on 127.0.0.1:8080
```

Send requests to your destination through this proxy:
```
curl https://testserver.com:5000 \
  --cacert test-ca/ca.cert \
  -x http://127.0.0.1:8080 --proxytunnel \
  --verbose
```

Please note this means you need to have a server running at `testserver.com:5000`, to do so, you can use the sample python server provided:

```
cd sample-server
pyenv local # 3.6.4 version
pip install flask

python main.py
```

You will then have a server running on `127.0.0.1:5000`. You can then point `testserver.com` to this server by editing your `/etc/hosts`:

```
127.0.0.1 testserver.com
```

Then you can try sending a request with a replacement:
```
curl 'https://testserver.com:5000?foo=foo' \
  --cacert test-ca/ca.cert \
  -x http://127.0.0.1:8080 --proxytunnel \
  --verbose
```

The python server will log:
```
GET https://testserver.com:5000/?bar=bar
```
