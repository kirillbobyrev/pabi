#include <string>
#include <vector>

#include "absl/flags/flag.h"
#include "absl/flags/parse.h"
#include "absl/flags/usage.h"

ABSL_FLAG(std::string, input_file, "commands.uci",
          "File with Universal Chess Engine (UCI) commands");

int main(int argc, char *argv[]) {
  const std::string kUsageMessage =
      "Run Aiseu Chess Engine on a set of UCI commands";
  absl::SetProgramUsageMessage(kUsageMessage);
  absl::ParseCommandLine(argc, argv);
}
