const DURATION = 3000;

function animateIn(el: HTMLElement): void {
  el.classList.remove('animate-exit');
  el.classList.add('animate-enter');
}

function animateOut(el: HTMLElement): void {
  el.classList.add('animate-exit');
  el.classList.remove('animate-enter');
}

function cycle(el: HTMLElement): void {
  animateIn(el);
  setTimeout(() => animateOut(el), DURATION);
}

function cycleAllElements(selector: string): void {
  const cycleElements = document.querySelectorAll<HTMLElement>(selector);
  cycleElements.forEach((el, index) => {
    setTimeout(() => {
      cycle(el);
      setInterval(() => cycle(el), cycleElements.length * DURATION);
    }, (index * DURATION));
  });
}

export {cycleAllElements}