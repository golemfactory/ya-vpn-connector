# ya-vpn-connector

This is tool used to connect TUN interface to yagna requestor websocket.

## Usage

After creating VPN network in yagna, you can use this tool to connect to it. 
Make sure that you are using the same network address as yagna.


If your network ID is dd45782a49374df98c9f6b94fd26702f

and your local VPN yagna network address is 192.168.8.1

and 192.168.8.7 is gateway node

Then you can connect to raw endpoint of this network using this tool:
export YAGNA_APPKEY=your_appkey
```
ya-vpn-connector -w ws://127.0.0.1:7465/net-api/v2/vpn/net/dd45782a49374df98c9f6b94fd26702f/raw/from/192.168.8.1/to/192.168.8.7
```

executable has to have NET_CAP capabilities to create interface (or be run as root)
```
sudo setcap 'cap_net_admin+ep' ya-vpn-connector
```

## Notes

This solution has some inefficiencies. 
If optimizing for performance it's good to get rid of whole codec stuff
or even move code to yagna client itself to get rid of extra websocket connection in the middle.

