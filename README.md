Making this to learn about dns and because NordVPN and systemd-resolved can't work together for 1 day.
I plan to use https://github.com/hickory-dns/hickory-dns libraries and https://github.com/EmilHernvall/dnsguide

Working:
 - Basic logging
 - systemd-service (It even pats the watchdog :))
 - Thanks hickory
   - Plain local DNS Server
   - Plain DNS resolving
   - dns over TLS resolving
   - dns over https and http/3 resolving
   - plain dns resolving
   - dns cache resolving
 
Todo (maybe): 
 - Config
   - routing system to point certain name queries to different resolving strategies.
     - Currently hardcoded for nordvpn to forward the queries to local dns servers
     - useful for allowing a VPN to query its server's ip since it only allows plain DNS packets.
 - packaging the thing
 - recursive dns resolving (I don't think this can be done over TLS?)
 - resolving strategies may have a fall-through system.
