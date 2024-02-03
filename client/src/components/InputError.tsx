export function InputError({ error }: { error: string }) {
  return <p class="pt-1 text-sm md:text-base text-red-500 ">{error}</p>;
}

export function GenericError({ error }: { error: string | null }) {
  return <p class="text-center text-sm md:text-base text-red-500 ">{error}</p>;
}
