export function PortalBrandMark() {
  return (
    <div className="flex h-8 w-8 items-center justify-center overflow-hidden rounded-xl bg-white/92 dark:bg-zinc-950/92 border border-zinc-200/80 shadow-[0_1px_0_rgba(15,23,42,0.04)] dark:border-zinc-800">
      <svg
        aria-hidden="true"
        className="h-5 w-5 text-zinc-950 dark:text-zinc-50"
        fill="none"
        viewBox="0 0 24 24"
        xmlns="http://www.w3.org/2000/svg"
      >
        <rect
          height="15"
          rx="4"
          stroke="currentColor"
          strokeWidth="1.8"
          width="15"
          x="4.5"
          y="4.5"
        />
        <path d="M12 7.5V10.2" stroke="currentColor" strokeLinecap="round" strokeWidth="1.8" />
        <path d="M12 13.8V16.5" stroke="currentColor" strokeLinecap="round" strokeWidth="1.8" />
        <path d="M7.5 12H10.2" stroke="currentColor" strokeLinecap="round" strokeWidth="1.8" />
        <path d="M13.8 12H16.5" stroke="currentColor" strokeLinecap="round" strokeWidth="1.8" />
        <circle cx="12" cy="12" fill="currentColor" r="1.7" />
      </svg>
    </div>
  );
}
