# Dropbox

## How to use

Using public `Docker` image
```
docker run -d -p $PORT:80 -v $DATA_DIR:/data tikhoplav/dropbox
```

<br>

## TODO in `v1.0.0`:
- [x] Add health check route;
- [x] Add `OPTION` handler;
- [x] Add multipart form upload handler;
- [x] Add verbose errors for upload;
- [x] Add `DELETE` handler;