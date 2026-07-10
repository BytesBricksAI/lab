use simplant_lab::Component as _;
use simplant_lab::external::re_sdk_types::try_serialize_field;

/// Custom archetype for rendering a 3D height field in the spatial view.
///
/// A height field is a 2D grid of height values stored as an image buffer,
/// with an optional colormap for GPU-side color mapping.
#[derive(Default)]
pub struct HeightField {
    pub buffer: Option<simplant_lab::SerializedComponentBatch>,
    pub format: Option<simplant_lab::SerializedComponentBatch>,
    pub colormap: Option<simplant_lab::SerializedComponentBatch>,
}

impl simplant_lab::Archetype for HeightField {
    fn name() -> simplant_lab::ArchetypeName {
        "HeightField".into()
    }

    fn display_name() -> &'static str {
        "Height Field"
    }

    fn required_components() -> std::borrow::Cow<'static, [simplant_lab::ComponentDescriptor]> {
        vec![Self::descriptor_buffer(), Self::descriptor_format()].into()
    }

    fn optional_components() -> std::borrow::Cow<'static, [simplant_lab::ComponentDescriptor]> {
        vec![Self::descriptor_colormap()].into()
    }
}

impl HeightField {
    /// Returns the [`simplant_lab::ComponentDescriptor`] for [`Self::buffer`].
    #[inline]
    pub fn descriptor_buffer() -> simplant_lab::ComponentDescriptor {
        simplant_lab::ComponentDescriptor {
            archetype: Some("HeightField".into()),
            component: "HeightField:buffer".into(),
            component_type: Some(simplant_lab::components::ImageBuffer::name()),
        }
    }

    /// Returns the [`simplant_lab::ComponentDescriptor`] for [`Self::format`].
    #[inline]
    pub fn descriptor_format() -> simplant_lab::ComponentDescriptor {
        simplant_lab::ComponentDescriptor {
            archetype: Some("HeightField".into()),
            component: "HeightField:format".into(),
            component_type: Some(simplant_lab::components::ImageFormat::name()),
        }
    }

    /// Returns the [`simplant_lab::ComponentDescriptor`] for [`Self::colormap`].
    #[inline]
    pub fn descriptor_colormap() -> simplant_lab::ComponentDescriptor {
        simplant_lab::ComponentDescriptor {
            archetype: Some("HeightField".into()),
            component: "HeightField:colormap".into(),
            component_type: Some(simplant_lab::components::Colormap::name()),
        }
    }

    /// Create a new `HeightField` from an image buffer and format.
    ///
    /// The image buffer contains the raw height data (e.g. F32 luminance),
    /// and the format describes its dimensions and channel type.
    #[inline]
    pub fn new(
        buffer: impl Into<simplant_lab::components::ImageBuffer>,
        format: impl Into<simplant_lab::components::ImageFormat>,
    ) -> Self {
        Self::default().with_buffer(buffer).with_format(format)
    }

    #[inline]
    pub fn with_buffer(mut self, buffer: impl Into<simplant_lab::components::ImageBuffer>) -> Self {
        self.buffer = try_serialize_field::<simplant_lab::components::ImageBuffer>(
            Self::descriptor_buffer(),
            [buffer.into()],
        );
        self
    }

    #[inline]
    pub fn with_format(mut self, format: impl Into<simplant_lab::components::ImageFormat>) -> Self {
        self.format = try_serialize_field::<simplant_lab::components::ImageFormat>(
            Self::descriptor_format(),
            [format.into()],
        );
        self
    }

    #[inline]
    #[expect(dead_code)] // Not used in this example, but could be useful for users of the archetype.
    pub fn with_colormap(
        mut self,
        colormap: impl Into<simplant_lab::components::Colormap>,
    ) -> Self {
        self.colormap = try_serialize_field::<simplant_lab::components::Colormap>(
            Self::descriptor_colormap(),
            [colormap.into()],
        );
        self
    }
}

impl simplant_lab::AsComponents for HeightField {
    #[inline]
    fn as_serialized_batches(&self) -> Vec<simplant_lab::SerializedComponentBatch> {
        [
            self.buffer.clone(),
            self.format.clone(),
            self.colormap.clone(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
