# flood-vfs-service
A background service which monitors torrents using [jesec/flood](https://github.com/jesec/flood) and refreshes the rclone-vfs cache using the [Remote Control API](https://rclone.org/rc/).

Useful when your torrent client is on a remote machine and your mount doesn't support polling (e.g. ssh).

Documentation TBD.
