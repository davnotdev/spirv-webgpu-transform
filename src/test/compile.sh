set -e

(cd splitcombined; ./compile.sh)
(cd splitdref; ./compile.sh)
(cd mirrorpatch; ./compile.sh)
(cd isnanisinfpatch; ./compile.sh)
(cd storagecubepatch; ./compile.sh)

