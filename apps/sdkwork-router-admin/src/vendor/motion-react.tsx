import * as React from 'react';

type MotionLikeProps = React.HTMLAttributes<HTMLElement> & {
  children?: React.ReactNode;
  layoutId?: string;
};

function createMotionElement(tagName: string) {
  return React.forwardRef<HTMLElement, MotionLikeProps>(function MotionElement(
    { layoutId: _layoutId, ...props },
    ref,
  ) {
    return React.createElement(tagName, { ...props, ref });
  });
}

export const motion = new Proxy(
  {},
  {
    get(_target, property: string) {
      return createMotionElement(property);
    },
  },
) as Record<string, React.ComponentType<MotionLikeProps>>;
