import React, { useEffect, useRef } from "react";
import * as THREE from "three";

// Real-world land/ocean mask, 2-degree resolution (91 lat rows x 180 lng
// cols), packed 8 cells per byte and base64-encoded. Generated once from
// an actual coastline dataset so continents are recognizable instead of
// hand-drawn ellipses; decoded once at module load.
const MASK_B64 =
  "//////////////////////////////////////////////////////////////////////////////////////////AD///////////////////////////+AAP//////4Bn////////////////8AAAD/////8AB4D///////////////4AAB//////zAAwf///////////////+AAAB//////4AAB///////////////8AAAAADAH///AAAB//////////////+AAAAAAAGABWAAAAf//////////////wAAAAAAAAAeAAAAAAIB//+f//////4AAAAAAAAAAEAAAAAAAAD/+B/////wAAAAAAAAAAACAAAAAAAAAMAAAGIAAAAAAAAAAAAAABwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4AAAAAAAAAAAAAAAAAAAAAAAAAAAABwgAAAAAAAAAAAAAAAAAAAAAAAAAAAB4AAAAAAAAAAAAAAAAAAAAAAAAAAAAA8AAAAAAAAAAAAAAAAAAAAAAAAAAAAB4AAAAAAAAAAAAAAAAAAAwAAAAAAAAA8AAAAAAAAAAAAAAAAAAAQAAAAAAAAB8AAAAAAAAAAAAAAAAAYAMAAAAAAAAA+AAAAAAAAAAAAAAAAAoACAAAAAAAAA/wAAAAAAAAAAAAAAABwADAAAAAAAAA/wAAAAAAAAAAAAAAAD8AEAAAAAAAAAf8AAAAADgAAAAAADAP8AAAAAAAAAAAf+AAAAAD4AAAAAAD+f+AAAAAAAAAAAf+AAAAAH8AAAAAAD//+AAAAAAAAAAAf/AAAAAP+AAAAAAD//+AAAAAAAAAAAf/AAAAAP+AAAAAAD//+AAAAAAAAAAAf/gAAAAP/DAAAAAH//8AAAAAAAAAAAf/8AAAAP/DgAAAAH//4AAAAAAAAAAAf/8AAAAf/BgAAAAA//4AAAAAAAAAAAf/+AAAA//hgAAAAAP/wABAAAAAAAAB//+AAAA//5gAAAAAH5gAAAAAAAAAAD//+AAAAf/4wAAAAABxAAAAAAAAAAAD///AAAAf/4AAAAAAARAAAAAAAAAAAH///AAAAf/wAAAAAAoAIAAAAAAAAAAH///gAAAf/wAAAAAMAHQAAAAAAAAAAP///gAAAf/wAAAABAAD0AAAAAAAAAAP//+AAAA//wAAAACAwPgAAAAAAAAAAP//wAAAB//4AAAAHOh0AAAAAAAAAAAP//AAAAB//8AAAAMegAAAAAAAAAAAAH/+AAAAB//+AAAAKOAAAAAAAAAAAAAD/+AAAAB///AAAAUmAAAAAAAAAAAAAD/wAAAfn///gAAAEBAAAAAAAAAAAAAH/gAAA/////gACAIAMAAAAAAAAAAAAheAAAB/////wAEABAAAAAAAAAAAAABgwAAAD////8AAMADgoAAAAAAAAAAAHgAAAAD////7AAOAPgAAAAAAAAAAAA8AAAAAD////74AeAfggAAAAAAAAAAH8AEAAAD////3+AfA/AQAAAAAAAAAAPGCAAAAD////n+Afx/AAAAAAAAAAAAPAoAAAAD////v/B///4AAAAAAAAAABfAAAAAAB////P2D///+AAAAAAAAAAC/AAAAAAB////fx/////AAAAAAAAAAF/AQAAAAA/////v/////gAAAAAAAAAD/8QAAAAAP////v/////gAAAAAAAAAP//wAAAAAP/zg///////gAAAAAAAAAf//8AAAAAH/AA///////g4AAAAAAAA///8AAAAAAfAA///////iOAAAAAAAB///+AAAAAPAZv///////GCAAAAAAAD///+AAAAAPoLv///////eCAAAAAAAD////gAAAAPwz4H//////+AAAAAAAAD////0AAAAA+v4P///////xgAAAAAAB////8gAAAA//9f///////5EAAAAAAD////4cAAAD///////////8AAAAAAAH////4AAAAAf//////////+AAAAAAAP///3/4AAAbn///////////AQAAAAA////n/wAAAZBv/////////6AwAACAA///+D/AAAACBx/////////4A8AAAgC///wD+AAAACA4f////////+AMAAf0P///gDgAAAAAH8D//////////EAAP/////wPAA4AAAH5///////////98AD/////908B8AcAA+/v///////////+f/////+h7B+geAAf/H///////////wP///4Zcj8D/wAAAP/xb/v////////AH/gYP8434F//gAAA4AAj///////wAAAAADfTC+AH/9gAAAAAMBi///+D4AAAAAADwBUAAP//gAAAAADAAv/gAIAAAAAAAA+xMAA///wAAAAAAcAB/4ABAAAAAAAADCg+////wAAdAAAAAAIAAAAAAAAAAAAC//h///0AALwAAAADgAAAAAAAAAAAAAD/+ev7UAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
const MASK_STEP = 2;
const MASK_LAT_N = 91;
const MASK_LNG_N = 180;
const maskBytes = Uint8Array.from(atob(MASK_B64), (c) => c.charCodeAt(0));

function isLand(lat, lng) {
  const latIdx = Math.min(MASK_LAT_N - 1, Math.max(0, Math.round((lat + 90) / MASK_STEP)));
  const lngIdx = Math.min(MASK_LNG_N - 1, Math.round((((lng + 180) % 360) + 360) % 360 / MASK_STEP));
  const k = latIdx * MASK_LNG_N + lngIdx;
  const byte = maskBytes[k >> 3];
  return ((byte >> (7 - (k % 8))) & 1) === 1;
}

function landPoints() {
  const pts = [];
  for (let lat = -85; lat <= 85; lat += 1) {
    for (let lng = -180; lng < 180; lng += 1) {
      if (isLand(lat, lng) && Math.random() > 0.1) pts.push({ lat, lng });
    }
  }
  return pts;
}

const RADIUS = 120;
function toVec3(lat, lng, r) {
  const phi = ((90 - lat) * Math.PI) / 180;
  const theta = ((lng + 180) * Math.PI) / 180;
  return new THREE.Vector3(
    -r * Math.sin(phi) * Math.cos(theta),
    r * Math.cos(phi),
    r * Math.sin(phi) * Math.sin(theta)
  );
}

export default function Globe({ connectedServer, status, accentColor = "#3ddc97" }) {
  const mountRef = useRef(null);
  const pointCloudRef = useRef(null);
  const cameraRef = useRef(null);
  const rendererRef = useRef(null);
  const sceneRef = useRef(null);
  const frameRef = useRef(null);
  const idleSpinRef = useRef(true);
  const markerGroupRef = useRef(null);

  useEffect(() => {
    const mount = mountRef.current;
    const width = mount.clientWidth;
    const height = mount.clientHeight;

    const scene = new THREE.Scene();
    sceneRef.current = scene;

    const camera = new THREE.PerspectiveCamera(50, width / height, 0.1, 2000);
    camera.position.set(0, 0, 340);
    cameraRef.current = camera;

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    mount.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    const pts = landPoints();
    const positions = new Float32Array(pts.length * 3);
    pts.forEach(({ lat, lng }, i) => {
      const v = toVec3(lat, lng, RADIUS);
      positions[i * 3] = v.x;
      positions[i * 3 + 1] = v.y;
      positions[i * 3 + 2] = v.z;
    });
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    const mat = new THREE.PointsMaterial({
      color: 0xffffff,
      size: 1.7,
      sizeAttenuation: true,
      transparent: false,
      opacity: 1,
      depthTest: true,
      depthWrite: true,
    });
    const pointCloud = new THREE.Points(geo, mat);
    pointCloudRef.current = pointCloud;
    scene.add(pointCloud);

    // Opaque core sphere, slightly smaller than the point radius, so the
    // far side of the globe is actually blocked from view instead of
    // bleeding through the front (e.g. Australia showing through Europe).
    const core = new THREE.Mesh(
      new THREE.SphereGeometry(RADIUS - 1.2, 48, 48),
      new THREE.MeshBasicMaterial({ color: 0x05060a })
    );
    pointCloud.add(core);

    const markerGroup = new THREE.Group();
    markerGroupRef.current = markerGroup;
    scene.add(markerGroup);

    scene.add(new THREE.AmbientLight(0xffffff, 1));

    const onResize = () => {
      const w = mount.clientWidth;
      const h = mount.clientHeight;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
    };
    window.addEventListener("resize", onResize);

    const clock = new THREE.Clock();
    const animate = () => {
      frameRef.current = requestAnimationFrame(animate);
      const dt = clock.getDelta();
      if (idleSpinRef.current) {
        pointCloud.rotation.y += dt * 0.08;
      }
      markerGroup.rotation.y = pointCloud.rotation.y;
      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(frameRef.current);
      window.removeEventListener("resize", onResize);
      mount.removeChild(renderer.domElement);
      renderer.dispose();
    };
  }, []);

  // fly camera to the connected server's lat/lng and drop an accent marker
  useEffect(() => {
    const camera = cameraRef.current;
    const markerGroup = markerGroupRef.current;
    if (!camera || !markerGroup) return;

    markerGroup.clear();

    if (status === "connected" && connectedServer) {
      idleSpinRef.current = false;

      const markerPos = toVec3(connectedServer.lat, connectedServer.lng, RADIUS + 1.5);
      const markerMesh = new THREE.Mesh(
        new THREE.SphereGeometry(2.4, 16, 16),
        new THREE.MeshBasicMaterial({ color: new THREE.Color(accentColor) })
      );
      markerMesh.position.copy(markerPos);
      markerGroup.add(markerMesh);

      const ring = new THREE.Mesh(
        new THREE.RingGeometry(3.2, 4.4, 32),
        new THREE.MeshBasicMaterial({
          color: new THREE.Color(accentColor),
          transparent: true,
          opacity: 0.55,
          side: THREE.DoubleSide,
        })
      );
      ring.position.copy(markerPos);
      ring.lookAt(markerPos.clone().multiplyScalar(2));
      markerGroup.add(ring);

      const target = toVec3(connectedServer.lat, connectedServer.lng, RADIUS);
      const to = target.clone().normalize().multiplyScalar(220);

      const duration = 1300;
      const start = performance.now();
      const from = camera.position.clone();
      const tick = (now) => {
        const t = Math.min(1, (now - start) / duration);
        const eased = 1 - Math.pow(1 - t, 3);
        camera.position.lerpVectors(from, to, eased);
        camera.lookAt(0, 0, 0);
        if (t < 1) requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    } else {
      idleSpinRef.current = true;
      const duration = 1000;
      const start = performance.now();
      const from = camera.position.clone();
      const to = new THREE.Vector3(0, 0, 340);
      const tick = (now) => {
        const t = Math.min(1, (now - start) / duration);
        const eased = 1 - Math.pow(1 - t, 3);
        camera.position.lerpVectors(from, to, eased);
        camera.lookAt(0, 0, 0);
        if (t < 1) requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    }
  }, [status, connectedServer, accentColor]);

  return <div ref={mountRef} className="globe-mount" />;
}
