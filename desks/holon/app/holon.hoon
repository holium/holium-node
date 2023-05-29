/+  dbug, default-agent, lib=holon-peers
|%
+$  card  card:agent:gall
+$  state-0  
  $:  %0
      ip=cord
  ==
--
=|  state-0
=*  state  -
%-  agent:dbug
=<
  ^-  agent:gall
  |_  =bowl:gall
  +*  this  .
      def   ~(. (default-agent this %|) bowl)
      hol   ~(. +> [bowl ~])
  ::
  ++  on-init
    ^-  (quip card _this)
    =.  ip.state  '127.0.0.1'
    `this
  ::
  ++  on-save  !>(state)
  ::
  ++  on-peek
    |=  =path
    ^-  (unit (unit cage))
    ?+    path  (on-peek:def path)
        [%x %peers ~]
      =/  peers  .^((map @p ?(%alien %known)) %ax [(scot %p our.bowl) %$ (scot %da now.bowl) %peers ~])
      ``noun+!>((view:enjs:lib [%peers peers]))
      ::
        [%x %peers @ ~]
      ?>  =(our.bowl src.bowl)
      =/  =ship  `@p`(slav %p i.t.t.path)
      =/  peer  .^(ship-state:ames %ax [(scot %p our.bowl) %$ (scot %da now.bowl) %peers (scot %p ship) ~])
      ?>  ?=(%known -.peer)   ::  if %known 
      =/  pstate      +.peer
      =/  route       route.pstate
      ?~  route       ~
      ~&  >>  [u.route]
      ?.  ?=(%.y direct.u.route)
        ~&  >>  ['Indirect route to %p' ship]
        =/  address   `@if`p.lane.u.route
        ~&  >>  [address]
        :: TODO lookup via galaxy
        ~
      ::  We have a direct route to the peer
      ?>  ?=(%.y direct.u.route)
      =/  address   `@if`p.lane.u.route
      ~&  >>  [address]
      :: ~
      ``noun+!>((view:enjs:lib [%peer peer=[ship address]]))
      ::
    ==
  ::
  ++  on-poke
    |=  [=mark =vase]
    ^-  (quip card _this)
    |^
    =^  cards  state
    ~&  >>  [mark vase]
    ?+  mark              (on-poke:def mark vase)
        %set-peer-info 
      =/  ip          !<(cord vase)
      =.  ip.state    ip
      `state
    ==
    [cards this]
    --
  ::
  ++  on-load  on-load:def
  ++  on-watch  on-watch:def
  ++  on-agent  on-agent:def
  ++  on-arvo   on-arvo:def
  ++  on-leave  on-leave:def
  ++  on-fail   on-fail:def
--
|_  [=bowl:gall cards=(list card)]
::
--