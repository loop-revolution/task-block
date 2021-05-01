use crate::blocks::task_block::TaskBlock;
use block_tools::{
	auth::{
		optional_token, optional_validate_token,
		permissions::{has_perm_level, PermLevel},
	},
	blocks::Context,
	display_api::component::{
		atomic::{
			icon::{Icon, IconComponent},
			text::TextComponent,
		},
		form::dropdown::DropdownComponent,
		interact::link::LinkComponent,
		layout::{
			card::CardComponent,
			stack::{AlignXOptions, StackComponent},
		},
		DisplayComponent,
	},
	models::Block,
	LoopError,
};

impl TaskBlock {
	pub fn handle_embed_display(
		block: &Block,
		context: &Context,
	) -> Result<DisplayComponent, LoopError> {
		let conn = &context.conn()?;
		let user_id = optional_validate_token(optional_token(context))?;

		let Self {
			name,
			description,
			status,
		} = Self::from_id(block.id, user_id, conn)?;

		let mut left_col = StackComponent::vertical();
		let mut middle_col = StackComponent::vertical();
		let mut right_col = StackComponent {
			align_x: Some(AlignXOptions::Right),
			..StackComponent::vertical()
		};

		let name = name
			.and_then(|block| block.block_data)
			.unwrap_or_else(|| "Untitled Habit".into());
		let text = TextComponent {
			bold: Some(true),
			..TextComponent::new(name)
		};
		let link = LinkComponent {
			app_path: Some(format!("/b/{}", block.id)),
			no_style: Some(true),
			..LinkComponent::new(text)
		};
		middle_col.push(link);

		if let Some(desc) = Self::description(&description, false) {
			middle_col.push(desc)
		}

		let mut status_dropdown = DropdownComponent {
			disabled: Some(true),
			..Self::status(status, block.id)
		};
		if let Some(user_id) = user_id {
			if has_perm_level(user_id, block, PermLevel::Edit) {
				status_dropdown.disabled = Some(false)
			}
		}
		right_col.push(status_dropdown);
		left_col.push(IconComponent::new(Icon::TaskComplete));

		let mut content = StackComponent::horizontal();
		content.push(left_col);
		content.push(middle_col);
		content.push(right_col);

		Ok(CardComponent {
			color: block.color.clone(),
			..CardComponent::new(content)
		}
		.into())
	}
}
