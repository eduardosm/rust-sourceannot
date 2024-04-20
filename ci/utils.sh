# shellcheck shell=bash

echo_stderr() {
  echo "$@" >&2
}

begin_group() {
  if [ $# -ne 1 ]; then
    echo_stderr "Invalid use of $0"
    exit 1
  fi
  echo "::group::$1"
}

end_group() {
  if [ $# -ne 0 ]; then
    echo_stderr "Invalid use of $0"
    exit 1
  fi
  echo "::endgroup::"
}
