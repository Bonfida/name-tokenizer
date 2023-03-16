import typescript from "@rollup/plugin-typescript";
import commonjs from "@rollup/plugin-commonjs";
import { terser } from "rollup-plugin-terser";

export default {
  input: "src/index.ts",
  output: [
    {
      file: "dist/index.mjs",
      format: "esm",
    },
    { file: "dist/index.cjs", format: "cjs" },
  ],
  plugins: [typescript(), commonjs(), terser()],
};
