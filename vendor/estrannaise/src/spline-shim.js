export default class Spline {
  constructor(xs, ys) {
    if (!Array.isArray(xs) || !Array.isArray(ys) || xs.length !== ys.length || xs.length < 2) {
      throw new Error('Spline requires equal-length arrays with at least two points');
    }
    // Ensure strictly increasing x; if not, sort pairs
    const pairs = xs.map((x, i) => [x, ys[i]]).sort((a, b) => a[0] - b[0]);
    this.x = pairs.map(p => p[0]);
    this.y = pairs.map(p => p[1]);

    const n = this.x.length;
    this.y2 = new Array(n).fill(0);
    const u = new Array(n - 1).fill(0);

    // Natural boundary conditions: second derivatives 0 at endpoints
    this.y2[0] = 0;
    u[0] = 0;

    for (let i = 1; i <= n - 2; i++) {
      const sig = (this.x[i] - this.x[i - 1]) / (this.x[i + 1] - this.x[i - 1]);
      const p = sig * this.y2[i - 1] + 2;
      this.y2[i] = (sig - 1) / p;
      const dy1 = (this.y[i + 1] - this.y[i]) / (this.x[i + 1] - this.x[i]);
      const dy0 = (this.y[i] - this.y[i - 1]) / (this.x[i] - this.x[i - 1]);
      u[i] = (6 * (dy1 - dy0) / (this.x[i + 1] - this.x[i - 1]) - sig * u[i - 1]) / p;
    }

    this.y2[n - 1] = 0;
    for (let k = n - 2; k >= 0; k--) {
      this.y2[k] = this.y2[k] * this.y2[k + 1] + u[k];
    }
  }

  at(xp) {
    const n = this.x.length;
    // Clamp xp to data range
    if (xp <= this.x[0]) return this.y[0];
    if (xp >= this.x[n - 1]) return this.y[n - 1];

    // Binary search to find interval
    let klo = 0;
    let khi = n - 1;
    while (khi - klo > 1) {
      const k = (khi + klo) >> 1;
      if (this.x[k] > xp) khi = k;
      else klo = k;
    }

    const h = this.x[khi] - this.x[klo];
    if (h === 0) return this.y[klo]; // guard

    const a = (this.x[khi] - xp) / h;
    const b = (xp - this.x[klo]) / h;

    return a * this.y[klo]
      + b * this.y[khi]
      + ((a * a * a - a) * this.y2[klo] + (b * b * b - b) * this.y2[khi]) * (h * h) / 6;
  }
}
