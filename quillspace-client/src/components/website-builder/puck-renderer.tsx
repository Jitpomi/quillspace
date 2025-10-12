/**
 * Puck Renderer for Published Pages
 * Renders Puck data as static HTML for published author (website-builder)
 */

/** @jsxImportSource react */
import { qwikify$ } from "@builder.io/qwik-react";
import { Render, type Data } from "@measured/puck";
import { puckConfig } from "./puck-config";

interface PuckRendererProps {
  data: Data;
}

// React component for Puck Renderer
const PuckRendererReact = ({ data }: PuckRendererProps) => {
  return (
    <div>
      <Render config={puckConfig} data={data} />
    </div>
  );
};

// Qwikified component
export const PuckRenderer = qwikify$(PuckRendererReact, {
  eagerness: "load",
});

export default PuckRenderer;
