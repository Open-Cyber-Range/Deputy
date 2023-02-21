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
	Video = "Video",
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

export interface Account {
	name: string;
	password: string;
}

export interface VM {
	accounts?: Account[];
	default_account?: string;
	operating_system: string;
	architecture: string;
	type: string;
	file_path: string;
	readme_path?: string;
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

export interface Video {
	file_path: string;
}

export interface Package {
	package: PackageBody;
	content: Content;
	virtual_machine?: VM;
	feature?: Feature;
	condition?: Condition;
	event?: Event;
	inject?: Inject;
	picture?: Picture;
	video?: Video;
}

