Making this to learn about dns and because NordVPN and systemd-resolved can't work together for 1 day.
I plan to use https://github.com/hickory-dns/hickory-dns libraries and https://github.com/EmilHernvall/dnsguide

Goal 1:
 - Plain local DNS Server
 - Plain DNS resolving
 - Logging
 - Config
 - systemd-service
 - packaging the thing

High level goals:
 - dns over TLS resolving
 - dns over https and http/3 resolving
 - plain dns resolving
 - dns cache resolving
 - recursive dns resolving (I don't think this can be done over TLS?)

 - routing system to point certain name queries to different resolving strategies.
   - useful for allowing a VPN to query its server's ip since it only allows plain DNS packets.
 - resolving strategies may have a fall-through system.