from .models import Email
from django.forms import ModelForm, TextInput, Textarea


class EmailForm(ModelForm):

	class Meta:
		model = Email
		fields = ['author', 'subject', 'message', 'date']
		widgets = {
		    'author': TextInput(attrs={
		        'class': 'form-control',
		        'placeholder': 'From:',
		    }),
		    'subject': TextInput(attrs={
		        'class': 'form-control',
		        'placeholder': 'Subject',
		    }),
		    'message': Textarea(attrs={
		        'class': 'form-control',
		        'placeholder': 'Message',
		    }),
		}