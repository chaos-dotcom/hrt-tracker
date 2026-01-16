export type EstrannaiseModel =
  | 'EB im'
  | 'EV im'
  | 'EEn im'
  | 'EC im'
  | 'EUn im'
  | 'EUn casubq'
  | 'patch tw'
  | 'patch ow';

export const PKParameters: Record<EstrannaiseModel, [number, number, number, number]> = {
  'EV im': [478.0, 0.236, 4.85, 1.24],
  'EEn im': [191.4, 0.119, 0.601, 0.402],
  'EC im': [246.0, 0.0825, 3.57, 0.669],
  'EB im': [1893.1, 0.67, 61.5, 4.34],
  'EUn im': [471.5, 0.01729, 6.528, 2.285],
  'EUn casubq': [16.15, 0.046, 0.022, 0.101],
  'patch tw': [16.792, 0.283, 5.592, 4.3],
  'patch ow': [59.481, 0.107, 7.842, 5.193],
};

function e2SteadyState3C(t: number, dose: number, T: number, d: number, k1: number, k2: number, k3: number) {
  return (
    dose *
    d *
    k1 *
    k2 *
    (Math.exp(-k1 * (t - T * Math.floor(t / T))) / (1 - Math.exp(-k1 * T)) / (k1 - k2) / (k1 - k3) -
      Math.exp(-k2 * (t - T * Math.floor(t / T))) / (1 - Math.exp(-k2 * T)) / (k1 - k2) / (k2 - k3) +
      Math.exp(-k3 * (t - T * Math.floor(t / T))) / (1 - Math.exp(-k3 * T)) / (k1 - k3) / (k2 - k3))
  );
}

function e2Curve3C(
  t: number,
  dose: number,
  d: number,
  k1: number,
  k2: number,
  k3: number,
  Ds = 0.0,
  D2 = 0.0,
  steadystate = false,
  T = 1.0,
) {
  if (!steadystate) {
    if (t < 0) {
      return 0;
    }

    let ret = 0;

    if (D2 > 0) {
      ret += D2 * Math.exp(-k3 * t);
    }

    if (Ds > 0) {
      if (k2 === k3) {
        ret += Ds * k2 * t * Math.exp(-k2 * t);
      } else {
        ret += Ds * k2 / (k2 - k3) * (Math.exp(-k3 * t) - Math.exp(-k2 * t));
      }
    }

    if (dose > 0 && d > 0) {
      if (k1 === k2 && k2 === k3) {
        ret += (dose * d * k1 * k1 * t * t * Math.exp(-k1 * t)) / 2;
      } else if (k1 === k2 && k2 !== k3) {
        ret +=
          (dose * d * k1 * k1 * (Math.exp(-k3 * t) - Math.exp(-k1 * t) * (1 + (k1 - k3) * t))) /
          (k1 - k3) /
          (k1 - k3);
      } else if (k1 !== k2 && k1 === k3) {
        ret +=
          (dose * d * k1 * k2 * (Math.exp(-k2 * t) - Math.exp(-k1 * t) * (1 + (k1 - k2) * t))) /
          (k1 - k2) /
          (k1 - k2);
      } else if (k1 !== k2 && k2 === k3) {
        ret +=
          (dose * d * k1 * k2 * (Math.exp(-k1 * t) - Math.exp(-k2 * t) * (1 - (k1 - k2) * t))) /
          (k1 - k2) /
          (k1 - k2);
      } else {
        ret +=
          dose *
          d *
          k1 *
          k2 *
          (Math.exp(-k1 * t) / (k1 - k2) / (k1 - k3) -
            Math.exp(-k2 * t) / (k1 - k2) / (k2 - k3) +
            Math.exp(-k3 * t) / (k1 - k3) / (k2 - k3));
      }
    }
    if (isNaN(ret)) {
      return 0;
    }
    return ret;
  }
  return e2SteadyState3C(t, dose, T, d, k1, k2, k3);
}

function PKFunctions(conversionFactor = 1.0) {
  return {
    'EV im': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['EV im'], 0.0, 0.0),
    'EEn im': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['EEn im'], 0.0, 0.0),
    'EC im': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['EC im'], 0.0, 0.0),
    'EUn im': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['EUn im'], 0.0, 0.0),
    'EUn casubq': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['EUn casubq'], 0.0, 0.0),
    'EB im': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['EB im'], 0.0, 0.0),
    'patch tw': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['patch tw'], 0.0, 0.0),
    'patch ow': (t: number, dose: number) => e2Curve3C(t, conversionFactor * dose, ...PKParameters['patch ow'], 0.0, 0.0),
  } as const;
}

export function e2multidose3C(
  t: number,
  doses: number[] = [1.0],
  times: number[] = [0.0],
  models: EstrannaiseModel[] = ['EV im'],
  conversionFactor = 1.0,
  intervals = false,
) {
  let computedTimes = times;
  if (intervals) {
    computedTimes = times.map((sum => value => (sum += value))(-times[0]));
  }

  let sum = 0;
  for (let i = 0; i < doses.length; i++) {
    const model = models[i];
    if (!model) continue;
    sum += PKFunctions(conversionFactor)[model](t - computedTimes[i], doses[i]);
  }
  return sum;
}
