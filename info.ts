export const info = async (specifier: string) => {
  const opts = {
    args: ["info", "--json", specifier],
    cwd: undefined as string | undefined,
    env: { DENO_NO_PACKAGE_JSON: "true" } as Record<string, string>,
    stdout: "piped",
    stderr: "inherit",
  };
  const output = await new Deno.Command(
    Deno.execPath(),
    opts as Deno.CommandOptions,
  ).output();
  if (!output.success) {
    throw new Error(`Failed to call 'deno info' on '${specifier}'`);
  }
  const txt = new TextDecoder().decode(output.stdout);
  console.log(txt);
  return JSON.parse(txt);
};
