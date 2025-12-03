# ensure-update

Automatically update a given git repository if enough time has passed since the last update.

```shell
$ ensure-update ~/some-repo-here --verbose --force
Already up to date.
```

By default, ensure-update will pull if eight hours have passed since the last time you ran it. This is customizable via command line args.

## What the hell do I do with this?

Whatever you like. For example, see this handy fish function:

```fish
function dl --description "download videos via yt-dlp"
    ensure-update /home/user/yt-dlp
    /home/user/yt-dlp/yt-dlp.sh $argv
end

```
