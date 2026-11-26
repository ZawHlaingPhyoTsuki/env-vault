import * as React from "react";
import { cn } from "@/lib/utils";

export const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, ...props }, ref) => (
    <input
      ref={ref}
      className={cn(
        "flex h-9 w-full border border-input bg-background px-3 text-[13px] text-foreground placeholder:text-muted-foreground transition-[border-color,color] duration-150 focus-visible:outline-1 focus-visible:outline-ring focus-visible:outline-offset-0 disabled:opacity-50",
        className,
      )}
      {...props}
    />
  ),
);
Input.displayName = "Input";
