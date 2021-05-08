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
			card::{CardComponent, DetachedMenu},
			stack::StackComponent,
		},
		menus::menu::MenuComponent,
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

		let mut icon_col = StackComponent::vertical();
		let mut content_col = StackComponent::vertical();

		let mut first_row = StackComponent::fit();

		let name = name
			.and_then(|block| block.block_data)
			.unwrap_or_else(|| "Untitled Task".into());
		let text = TextComponent {
			bold: Some(true),
			..TextComponent::new(name)
		};
		let link = LinkComponent {
			app_path: Some(format!("/b/{}", block.id)),
			no_style: Some(true),
			..LinkComponent::new(text)
		};
		first_row.push(link);

		let mut status_dropdown = DropdownComponent {
			disabled: Some(true),
			..Self::status(status, block.id)
		};

		let mut detached_menu = None;

		if let Some(user_id) = user_id {
			if has_perm_level(user_id, block, PermLevel::Edit) {
				status_dropdown.disabled = Some(false)
			}
			let mut menu = MenuComponent::from_block(block, user_id);
			menu.load_comments(conn)?;
			detached_menu = Some(DetachedMenu::bottom_right(menu));
		}
		first_row.push(status_dropdown);
		content_col.push(first_row);

		if let Some(desc) = Self::description(&description, false) {
			content_col.push(desc)
		}

		icon_col.push(IconComponent::new(Icon::TaskComplete));

		let mut content = StackComponent::horizontal();
		content.push(icon_col);
		content.push(content_col);

		Ok(CardComponent {
			color: block.color.clone(),
			detached_menu,
			..CardComponent::new(content)
		}
		.into())
	}
}
