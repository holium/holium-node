/-  store=holon
=<  [store .]
=,  store
|%
++  enjs
  =,  enjs:format
  |%
  ++  view
    |=  view=^view
    ^-  json
    %-  pairs
    :_  ~
    ^-  [cord json]
    :-  -.view  
    ?-  -.view
        %peers
      (peers-table peers.view)
      ::
        %peer
      (peer-info peer.view)
    ==
    ::
    ++  peers-table
      |=  peers=peers-table:store
      ^-  json
      %-  pairs
      %+  turn  ~(tap by peers)
      |=  [=^ship status=peer-status:store]
      ^-  [cord json]
      [(scot %p ship) s+status]
    ::
    ++  peer-info
      |=  peer=peer-info:store
      ^-  json
      %-  pairs
      =/  parsed-address    ~(rend co [%$ p=[p=%if q=address.peer]])
      :~
        ['patp' s+(scot %p id.peer)]
        ['ipv4' s+(crip (tail parsed-address))]
      ==
     
    --

::
--