/+  dbug, default-agent
|%
+$  card  card:agent:gall
--
%-  agent:dbug
=<
  ^-  agent:gall
  |_  =bowl:gall
  +*  this  .
      def   ~(. (default-agent this %|) bowl)
      hol   ~(. +> [bowl ~])
  ::
  ++  on-peek
    |=  =path
    ^-  (unit (unit cage))
    ?+    path  (on-peek:def path)
        [%x %peers ~]
      =/  peers  .^((map @p ?(%alien %known)) %ax [(scot %p our.bowl) %$ (scot %da now.bowl) %peers ~])
      ~&  >>  [peers]
      ::  for all peers, scry peers/@p
      =/  peer-ips  
        %+  turn  ~(tap by peers)
          |=  [patp=ship status=?(%alien %known)]
          :: scry peers/@p
          =/  peer  .^(ship-state:ames %ax [(scot %p our.bowl) %$ (scot %da now.bowl) %peers (scot %p patp) ~])
          :: ?:  =
          ::  TODO get peer route from ames  
          ::  check if route is alive or dead
          ::  if alive, get IP address
          ::  route=[~ [direct=%.y lane=[%.n p=0x8cf7.a45a.8972]]]
          ~&  >>  [peer]
          [patp status]
      :: ~&  >>  [peer-ips]
      ~
    ==
  ++  on-init  on-init:def 
  ++  on-save  on-save:def
  ++  on-load  on-load:def
  ++  on-poke  on-poke:def
  ++  on-watch  on-watch:def
  ++  on-agent  on-agent:def
  ++  on-arvo   on-arvo:def
  ++  on-leave  on-leave:def
  ++  on-fail   on-fail:def
--
|_  [=bowl:gall cards=(list card)]
::
--