test -z $DEVBOX_COREPACK_ENABLED || corepack enable --install-directory "/Users/micro/p/gh/levonk/ai-dashboard/.devbox/virtenv/nodejs/corepack-bin/"
test -z $DEVBOX_COREPACK_ENABLED || export PATH="/Users/micro/p/gh/levonk/ai-dashboard/.devbox/virtenv/nodejs/corepack-bin/:$PATH"
just bootstrap-internal