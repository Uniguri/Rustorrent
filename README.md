# Rustorrent

Torrent Client with rust

## Todo (TBD)

- [ ] [Bencode](https://en.wikipedia.org/wiki/Bencode#Encoding_algorithm) decoder
- [ ] Decoded Torrent file parser
- [ ] Get peer information from tracker
  - [ ] Using HTTP
  - [ ] Using UDP
- [ ] Receive pieces from peer
  - [ ] Handshake
  - [ ] Receive pieces
- [ ] Send pieces to peer
  - [ ] Waiting handshake
  - [ ] Send pieces
- [ ] Combine pieces into whole file
- [ ] Addtional
  - [ ] Implement using UDP
  - [ ] Implement using asynchronization

# Reference

- <http://bittorrent.org/beps/bep_0000.html>
- <https://wiki.theory.org//BitTorrentSpecification#>
- <https://www.nxted.co.jp/blog/blog_detail?id=40>
- <https://blog.jse.li/posts/torrent/>
