export default function InputError({ error }: { error: string }) {
  return (
    <div class="pt-1 text-sm text-red-500 dark:text-red-400 md:text-base">
      {error}
    </div>
  );
}
