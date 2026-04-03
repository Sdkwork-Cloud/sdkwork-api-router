import {
  forwardRef,
  type ComponentPropsWithoutRef,
  type ForwardedRef,
} from 'react';

type MotionDivProps = ComponentPropsWithoutRef<'div'> & {
  animate?: unknown;
  exit?: unknown;
  initial?: unknown;
  layout?: boolean | 'position' | 'size';
  transition?: unknown;
};

type MotionSectionProps = ComponentPropsWithoutRef<'section'> & {
  animate?: unknown;
  exit?: unknown;
  initial?: unknown;
  layout?: boolean | 'position' | 'size';
  transition?: unknown;
};

function MotionDiv(
  { animate: _animate, exit: _exit, initial: _initial, layout: _layout, transition: _transition, ...props }: MotionDivProps,
  ref: ForwardedRef<HTMLDivElement>,
) {
  return <div {...props} ref={ref} />;
}

function MotionSection(
  {
    animate: _animate,
    exit: _exit,
    initial: _initial,
    layout: _layout,
    transition: _transition,
    ...props
  }: MotionSectionProps,
  ref: ForwardedRef<HTMLElement>,
) {
  return <section {...props} ref={ref} />;
}

const MotionDivComponent = forwardRef<HTMLDivElement, MotionDivProps>(MotionDiv);
const MotionSectionComponent = forwardRef<HTMLElement, MotionSectionProps>(MotionSection);

MotionDivComponent.displayName = 'MotionDiv';
MotionSectionComponent.displayName = 'MotionSection';

export const motion = {
  div: MotionDivComponent,
  section: MotionSectionComponent,
};
