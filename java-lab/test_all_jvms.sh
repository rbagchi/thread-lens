set -e

for jvm in java-app-openjdk8 java-app-openjdk11 java-app-openjdk17 java-app-openjdk21 java-app-ibm17; do
  echo "--- Running test for $jvm ---"
  ./test_single_jvm.sh "$jvm"
  echo "--- Finished test for $jvm ---"
done
