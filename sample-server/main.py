from flask import Flask, request
app = Flask(__name__)
app.debug = True

@app.route('/')
def hello_world():
    print(f"{request.method} {request.url}")
    return 'Hello, World!'

app.run(
    debug=True,
    ssl_context=('fullchain.pem', 'key.pem'),
    port=5000
)
