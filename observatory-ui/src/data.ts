export type CMCAObject = {
  id: string;
  type: 'Artifact' | 'Activity' | 'Value';
  state: 'MEASURED' | 'PROPOSED' | 'ADMITTED' | 'CERTIFIED' | 'ACTIVE' | 'FROZEN';
  learnerState?: 'Active' | 'Gated' | 'Candidate';
  kappaQ: number; // Measurement scale
  uncertainty: number; // 0.0 to 1.0
  consequenceMass: number;
  position: [number, number, number];
};

const generateMockData = (): CMCAObject[] => {
  const data: CMCAObject[] = [];
  const types: ('Artifact' | 'Activity' | 'Value')[] = ['Artifact', 'Activity', 'Value'];
  const states: CMCAObject['state'][] = ['MEASURED', 'PROPOSED', 'ADMITTED', 'CERTIFIED', 'ACTIVE', 'FROZEN'];
  const learnerStates: CMCAObject['learnerState'][] = ['Active', 'Gated', 'Candidate'];

  for (let i = 0; i < 200; i++) {
    const type = types[Math.floor(Math.random() * types.length)];
    const state = states[Math.floor(Math.random() * states.length)];
    const hasLearnerState = ['Artifact', 'Activity'].includes(type);
    
    // Spread data out in 3D space based on type
    const layerOffset = type === 'Artifact' ? -100 : type === 'Activity' ? 0 : 100;
    
    data.push({
      id: `node-${i}`,
      type,
      state,
      learnerState: hasLearnerState ? learnerStates[Math.floor(Math.random() * learnerStates.length)] : undefined,
      kappaQ: Math.random() * 1000,
      uncertainty: Math.random() * 0.8 + 0.1, // 0.1 to 0.9
      consequenceMass: Math.random() * 500,
      position: [
        (Math.random() - 0.5) * 800,
        (Math.random() - 0.5) * 800,
        layerOffset + (Math.random() - 0.5) * 50
      ]
    });
  }
  return data;
};

export const MOCK_DATA = generateMockData();
