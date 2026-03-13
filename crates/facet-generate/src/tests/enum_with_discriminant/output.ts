export type Effect = 
    | { name: "temperature", attributes: ColorTemperatureAttributes }
    | { name: "contrast", attributes: ContrastAttributes }
    | { name: "exposure", attributes: ExposureAttributes };

export type EffectName = Effect["name"];

