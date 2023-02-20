export interface PackageMetadata {
	name: string;
	description: string;
	version: string;
	license: string;
	readme: string;
	readme_html: string;
	created_at: string;
}

export enum ContentType {
	VM = "VM",
	Feature = "Feature",
	Condition = "Condition",
	Inject = "Inject",
	Event = "Event",
	Picture = "Picture",
}

export enum FeatureType {
	Service = "Service",
	Configuration = "Configuration",
	Artifact = "Artifact",
}

export interface PackageBody {
	name: string;
	description: string;
	version: string;
	authors: string[];
	license: string;
	readme: string;
}

export interface Content {
	type: ContentType;
}

export interface VM {
	accounts?: any;
	default_account?: any;
	operating_system: string;
	architecture: string;
	type: string;
	file_path: string;
	readme_path?: any;
}

export interface Feature {
	feature_type: FeatureType;
	action: string | null;
	assets: string[][];
}

export interface Condition {
	action: string;
	assets: string[][];
	interval: number;
}

export interface Event {
	action: string;
	assets: string[][];
}

export interface Inject {
	action: string;
	assets: string[][];
}

export interface Picture {
	file_path: string;
}

export interface Package {
	package: PackageBody;
	content: Content;
	virtual_machine: VM | null;
	feature: Feature | null;
	condition: Condition | null;
	event: Event | null;
	inject: Inject | null;
	picture: Picture | null;
}

