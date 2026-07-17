import React, { useState, useEffect } from 'react';
import { ObservatoryMap } from './ObservatoryMap';
import { Legend } from './Legend';
import { generateMockData, DataPoint } from './mockData';
import './App.css';

function App() {
  const [data, setData] = useState<DataPoint[]>([]);

  useEffect(() => {
    setData(generateMockData());
  }, []);

  return (
    <div style={{ width: '100vw', height: '100vh', backgroundColor: '#020205', color: '#0ff', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      <header style={{ padding: '15px 20px', borderBottom: '1px solid rgba(0, 255, 255, 0.3)', display: 'flex', justifyContent: 'space-between', alignItems: 'center', backgroundColor: '#050510' }}>
        <h1 style={{ margin: 0, fontSize: '20px', textTransform: 'uppercase', letterSpacing: '2px', textShadow: '0 0 10px #0ff' }}>
          CMCA-RDF κ_q Observatory
        </h1>
        <div style={{ fontSize: '12px', opacity: 0.8 }}>STATUS: CALIBRATING...</div>
      </header>
      
      <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '1fr 1fr', gridTemplateRows: '1fr 1fr', gap: '2px', backgroundColor: 'rgba(0,255,255,0.2)' }}>
        <ObservatoryMap data={data} qValue={-1} width="100%" height="100%" />
        <ObservatoryMap data={data} qValue={0} width="100%" height="100%" />
        <ObservatoryMap data={data} qValue={1} width="100%" height="100%" />
        <ObservatoryMap data={data} qValue={2} width="100%" height="100%" />
      </div>
      
      <Legend />
    </div>
  );
}

export default App;
