CGI runner
==========

A Docker image with Apache2 that runs cgi-bin applications.


Build the image
---------------

```bash
docker build -f Dockerfile -t cgirunner:latest .
```

Run the server
--------------
With `--restart=always` Docker will make sure to restart this container after
the host OS is rebooted.

```bash
docker run --restart=always -d \
  -p 8080:80 \
  --hostname example.com \
  --name cgirunner \
  -v "$PWD":/usr/lib/cgi-bin \
  cgirunner:latest
```
