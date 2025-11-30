## Audio Converter API

**Run API** 

1 Go to the your project directory and ```cargo build ```

2 ``` cargo run```

3 Testing with Curl
```
curl -X POST http://localhost:3000/api/audio/convert \
  -F "file=@input.mp3" \
  -F "format=wav"
```
More features will be coming soon...
