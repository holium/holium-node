## Passport module

### States
- embryo: pre-verified identity
- alive: verified identity and ready to use keys
- void: if the keys are deregistered or epoch is incremented.

embryo -> alive
alive -> void
void -> embryo