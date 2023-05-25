|%
+$  peer-status   ?(%alien %known)
+$  peers-table   (map ship peer-status)
+$  peer-info     [id=@p address=@if]

+$  view
  $%  [%peers peers=peers-table]
      [%peer peer=peer-info]
      :: TODO more
  ==
--