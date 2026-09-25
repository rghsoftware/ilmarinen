# Sourced by Ilmarinen hooks (decision 0014): Ilmarinen is dormant unless the
# directory, or its nearest ancestor holding .ilmarinen.version, is initialized
# and has no .ilmarinen.off. ilm_active <dir> sets ILM_ROOT and returns 0 when active.
ilm_active() {
  d=$1
  while [ -n "$d" ]; do
    if [ -f "$d/.ilmarinen.version" ]; then
      [ -f "$d/.ilmarinen.off" ] && return 1
      ILM_ROOT=$d; return 0
    fi
    [ "$d" = / ] && return 1
    d=$(dirname "$d")
  done
  return 1
}
