export type DataPoint = {
  id: string;
  position: [number, number];
  q: number;
  kappa: number;
  n_eff: number;
  learnerState: 'ACTIVE' | 'GATED' | 'CANDIDATE';
  status: 'VALID' | 'DRIFTING' | 'SUPPORT_INVALID' | 'NUMERICALLY_UNCERTAIN';
  measurementStanding: 'MEASURED' | 'PROPOSED' | 'ADMITTED' | 'CERTIFIED' | 'ACTIVE' | 'FROZEN';
  gamma_min_plus: number;
};

export const generateMockData = (): DataPoint[] => {
  const data: DataPoint[] = [];
  const q_values = [-1, 0, 1, 2];
  const states: DataPoint['learnerState'][] = ['ACTIVE', 'GATED', 'CANDIDATE'];
  const standings: DataPoint['measurementStanding'][] = ['MEASURED', 'PROPOSED', 'ADMITTED', 'CERTIFIED', 'ACTIVE', 'FROZEN'];
  const statuses: DataPoint['status'][] = ['VALID', 'DRIFTING', 'SUPPORT_INVALID', 'NUMERICALLY_UNCERTAIN'];

  let idCounter = 0;
  for (const q of q_values) {
    for (let x = -10; x <= 10; x += 1.5) {
      for (let y = -10; y <= 10; y += 1.5) {
        if (Math.random() > 0.8) continue;
        data.push({
          id: `cell-${idCounter++}`,
          position: [x + Math.random() * 0.5, y + Math.random() * 0.5],
          q,
          kappa: Math.random() * 100,
          n_eff: Math.random() * 100,
          learnerState: states[Math.floor(Math.random() * states.length)],
          measurementStanding: standings[Math.floor(Math.random() * standings.length)],
          status: Math.random() > 0.8 ? statuses[Math.floor(Math.random() * 3) + 1] : 'VALID',
          gamma_min_plus: Math.random(),
        });
      }
    }
  }
  return data;
};
