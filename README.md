# filesink

Easy file sinkronization.

This tool lets authorized users upload files to predetermined locations on the server,
and can run a command whenever an upload happens.

## Usage
Write a config.yaml file like this:

```yaml
test.txt:
  auth: bratwurst
  cmd: base64 test.txt > wurst64.txt
  target: wurst.txt
logfile.log:
  auth: 65jP2AxZ7qnaI9gwtIp8K03YfGRQK5MG4Bp1AeIy7zLPRuzhXpAqEqae0M7wmDmP
  cmd: merge-logs.sh
```

then run filesink like this:

```
$ filesink config.yaml
```

it will then listen on port 8228 for uploads.
Try it out with this command:

```
$ curl -X PUT 'http://localhost:8228/test.txt?bratwurst' -d "mmmmhhh, lecker!"
```

## Notes on stability and security
Currently, there is probably a race condition when writing files and executing commands.
Also, long-running commands will slow the server down quite a lot, so don't run those.

The authorizaton code is not encrypted in the config file and there is no way to make it so, currently.

## LICENSE
filesink is provided under the MIT license. See [LICENSE](LICENSE).
