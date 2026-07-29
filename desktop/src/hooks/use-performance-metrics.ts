import { useEffect, useRef, useState } from "react";

export function useFramesPerSecond(): number {
  const [fps, setFps] = useState(0);
  const frameCount = useRef(0);

  useEffect(() => {
    let animationFrame = 0;
    let measuredAt = performance.now();

    const measure = (now: number) => {
      frameCount.current += 1;
      if (now - measuredAt >= 1_000) {
        setFps(Math.round((frameCount.current * 1_000) / (now - measuredAt)));
        frameCount.current = 0;
        measuredAt = now;
      }
      animationFrame = requestAnimationFrame(measure);
    };

    animationFrame = requestAnimationFrame(measure);
    return () => cancelAnimationFrame(animationFrame);
  }, []);

  return fps;
}

